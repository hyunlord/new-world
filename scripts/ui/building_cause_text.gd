# Building identity/provenance fields come from get_building_cause.
extends RefCounted

const Chronicle = preload("res://scripts/ui/chronicle_text.gd")


static func lines(read: Dictionary, locale: Node, info: Dictionary) -> Array[String]:
	if not bool(read.get("valid", false)):
		return [Chronicle.loc(locale, "BUILD_CAUSE_UNAVAILABLE")]
	var out: Array[String] = []
	# These are historical command-generation fields, never looked up from the current site.
	var template_name: String = Chronicle.loc(locale, str(read.get("name_key", "UI_UNKNOWN")))
	out.append(Chronicle.loc(locale, "BUILD_CAUSE_TITLE_FMT").format({"name": template_name, "tier": int(read.get("tier", 0))}))
	var kind: String = str(read.get("cause_kind", ""))
	var cause_key: String = ""
	match kind:
		"command": cause_key = "BUILD_CAUSE_COMMAND"
		"zone": cause_key = "BUILD_CAUSE_ZONE"
		"steward": cause_key = "BUILD_CAUSE_STEWARD"
		_: return [Chronicle.loc(locale, "BUILD_CAUSE_UNAVAILABLE")]
	out.append(Chronicle.loc(locale, cause_key))
	out.append(Chronicle.loc(locale, "BUILD_CAUSE_ORDER_FMT").format({
		"day": Chronicle.day(int(read.get("tick", 0)), info),
		"action": Chronicle.loc(locale, str(read.get("command_key", "UI_UNKNOWN"))),
		"x": int(read.get("x", -1)), "y": int(read.get("y", -1))}))
	return out
