extends RefCounted

## [Boyd & Richerson 1985, Henrich 2004, Tainter 1988]
## Six categories of knowledge persistence, from easiest to hardest to maintain.
## Each type has different transmission methods, practitioner requirements,
## regression grace periods, and rediscovery bonuses.

enum Type {
	UNIVERSAL_INTUITIVE,    ## 0: Fire, basic stone tools — nearly impossible to lose
	COMMON_PRACTICE,        ## 1: Basic farming, pottery — needs a community of practitioners
	CRAFT_TACIT,            ## 2: Metallurgy, advanced woodwork — master-apprentice chain required
	ELITE_SCHOLARLY,        ## 3: Writing, math, astronomy — institutional support needed
	INSTITUTIONAL_STATE,    ## 4: Law codes, bureaucracy, taxation — collapses with institutions
	INFRASTRUCTURE_NETWORKED ## 5: Trade networks, canal systems — needs physical infrastructure
}

## String ID used in JSON files -> enum mapping
const TYPE_FROM_STRING: Dictionary = {
	"universal_intuitive": Type.UNIVERSAL_INTUITIVE,
	"common_practice": Type.COMMON_PRACTICE,
	"craft_tacit": Type.CRAFT_TACIT,
	"elite_scholarly": Type.ELITE_SCHOLARLY,
	"institutional_state": Type.INSTITUTIONAL_STATE,
	"infrastructure_networked": Type.INFRASTRUCTURE_NETWORKED,
}

## Enum -> JSON string (for serialization)
const TYPE_TO_STRING: Dictionary = {
	Type.UNIVERSAL_INTUITIVE: "universal_intuitive",
	Type.COMMON_PRACTICE: "common_practice",
	Type.CRAFT_TACIT: "craft_tacit",
	Type.ELITE_SCHOLARLY: "elite_scholarly",
	Type.INSTITUTIONAL_STATE: "institutional_state",
	Type.INFRASTRUCTURE_NETWORKED: "infrastructure_networked",
}

## Default knowledge type per tech tier (can be overridden per-tech)
const TIER_DEFAULTS: Dictionary = {
	0: Type.UNIVERSAL_INTUITIVE,
	1: Type.COMMON_PRACTICE,
	2: Type.CRAFT_TACIT,
	3: Type.ELITE_SCHOLARLY,
	4: Type.INSTITUTIONAL_STATE,
}

## ---- Static configuration per KnowledgeType ----
## Used by TechMaintenanceSystem (C-1c) for regression calculations
const CONFIG: Dictionary = {
	Type.UNIVERSAL_INTUITIVE: {
		"min_practitioners": 1,
		"regression_grace_years": 50,
		"rediscovery_bonus": 3.0,
		"transmission_years": 0.2,
		"memory_decay_rate": 0.01,
		"carrier_model": "any_adult",
		"locale_key": "KNOWLEDGE_TYPE_UNIVERSAL_INTUITIVE",
	},
	Type.COMMON_PRACTICE: {
		"min_practitioners": 5,
		"regression_grace_years": 15,
		"rediscovery_bonus": 2.0,
		"transmission_years": 2.0,
		"memory_decay_rate": 0.03,
		"carrier_model": "many_practitioners",
		"locale_key": "KNOWLEDGE_TYPE_COMMON_PRACTICE",
	},
	Type.CRAFT_TACIT: {
		"min_practitioners": 2,
		"regression_grace_years": 6,
		"rediscovery_bonus": 1.6,
		"transmission_years": 4.0,
		"memory_decay_rate": 0.05,
		"carrier_model": "master_apprentice_chain",
		"locale_key": "KNOWLEDGE_TYPE_CRAFT_TACIT",
	},
	Type.ELITE_SCHOLARLY: {
		"min_practitioners": 3,
		"regression_grace_years": 4,
		"rediscovery_bonus": 1.4,
		"transmission_years": 6.0,
		"memory_decay_rate": 0.07,
		"carrier_model": "specialist_class",
		"locale_key": "KNOWLEDGE_TYPE_ELITE_SCHOLARLY",
	},
	Type.INSTITUTIONAL_STATE: {
		"min_practitioners": 6,
		"regression_grace_years": 2,
		"rediscovery_bonus": 1.3,
		"transmission_years": 8.0,
		"memory_decay_rate": 0.10,
		"carrier_model": "positions_and_records",
		"locale_key": "KNOWLEDGE_TYPE_INSTITUTIONAL_STATE",
	},
	Type.INFRASTRUCTURE_NETWORKED: {
		"min_practitioners": 4,
		"regression_grace_years": 3,
		"rediscovery_bonus": 1.5,
		"transmission_years": 5.0,
		"memory_decay_rate": 0.08,
		"carrier_model": "specialists_plus_infrastructure",
		"locale_key": "KNOWLEDGE_TYPE_INFRASTRUCTURE_NETWORKED",
	},
}


## Resolve knowledge type from a tech definition dict.
## Priority: explicit "knowledge_type" field > tier-based default > COMMON_PRACTICE fallback.
static func resolve_from_def(tech_def: Dictionary) -> int:
	var explicit: String = tech_def.get("knowledge_type", "")
	if explicit != "" and explicit in TYPE_FROM_STRING:
		return TYPE_FROM_STRING[explicit]
	var tier: int = tech_def.get("tier", 1)
	return TIER_DEFAULTS.get(tier, Type.COMMON_PRACTICE)
