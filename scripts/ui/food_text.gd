# Formats authoritative food observations; freshness and food choice remain in Rust.
extends RefCounted
const EconomyText = preload("res://scripts/ui/economy_text.gd")

static func person(locale: Node, data: Dictionary) -> String:
	var meals: PackedInt64Array = data.get("meal_kinds", PackedInt64Array())
	var names: PackedStringArray = []
	for kind: int in meals:
		names.append(EconomyText.item(locale, kind))
	var groups: PackedStringArray = []
	for group: String in data.get("groups", PackedStringArray()):
		groups.append(EconomyText.ltr(locale, "M6A_GROUP_" + group.to_upper()))
	return EconomyText.trf(locale, "M6A_MEALS", {"meals": ", ".join(names) if not names.is_empty() else EconomyText.ltr(locale, "M6A_NONE"), "groups": ", ".join(groups) if not groups.is_empty() else EconomyText.ltr(locale, "M6A_NONE")})

static func stacks(locale: Node, data: Dictionary) -> PackedStringArray:
	var result: PackedStringArray = []
	var kinds: PackedInt64Array = data.get("kinds", PackedInt64Array())
	var counts: PackedInt64Array = data.get("counts", PackedInt64Array())
	var days: PackedInt64Array = data.get("remaining_days", PackedInt64Array())
	var storage: PackedStringArray = data.get("storage", PackedStringArray())
	for i: int in mini(mini(kinds.size(), counts.size()), mini(days.size(), storage.size())):
		var remaining: String = EconomyText.trf(locale, "M6A_REMAINING", {"days": days[i]}) if days[i] >= 0 else EconomyText.ltr(locale, "UI_UNKNOWN")
		result.append(EconomyText.trf(locale, "M6A_STACK", {"item": EconomyText.item(locale, kinds[i]), "count": counts[i], "storage": EconomyText.ltr(locale, "M6A_STORAGE_" + storage[i].to_upper()), "remaining": remaining}))
	return result
