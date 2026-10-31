# diagnosis_text.gd — formatting helpers for M-READ diagnosis, why-not and requests.
# Purpose: pure formatting; callers supply the data dictionaries from the bridge.
# Bridge methods: none (this file never calls the bridge).
const KoreanText = preload("res://scripts/ui/mux/korean_text.gd")
const Names = preload("res://scripts/ui/person_names.gd")


## The first failing check's text from a diagnosis dictionary (the headline).
static func headline(checks: Dictionary, locale: Node) -> String:
	var text_keys: PackedStringArray = checks.get("text_keys", PackedStringArray())
	var params_array: Array = checks.get("params", [])
	if text_keys.is_empty():
		return ""
	return _resolve_text(locale, text_keys[0], params_array[0] if not params_array.is_empty() else {})


## The headline + next cause (one chain step) as "headline ← next".
static func headline_and_next(checks: Dictionary, locale: Node) -> String:
	var text_keys: PackedStringArray = checks.get("text_keys", PackedStringArray())
	var params_array: Array = checks.get("params", [])
	var chains: Array = checks.get("chains", [])
	if text_keys.is_empty():
		return ""
	var head: String = _resolve_text(locale, text_keys[0], params_array[0] if not params_array.is_empty() else {})
	if chains.is_empty():
		return head
	var chain: Array = chains[0] if chains[0] is Array else []
	if chain.is_empty():
		return head
	var next_entry: Dictionary = chain[0] if chain[0] is Dictionary else {}
	var next_text: String = _resolve_text(locale, str(next_entry.get("text_key", "")), next_entry.get("params", {}))
	if next_text.is_empty():
		return head
	return head + _loc(locale, "READ_DIAG_CHAIN_SEP") + next_text


## Format a full diagnosis chain as "headline ← next ← ..." from settlement/person diagnosis data.
static func chain_text(checks: Dictionary, index: int, locale: Node) -> String:
	var text_keys: PackedStringArray = checks.get("text_keys", PackedStringArray())
	var params_array: Array = checks.get("params", [])
	var chains: Array = checks.get("chains", [])
	if index < 0 or index >= text_keys.size():
		return ""
	var head: String = _resolve_text(locale, text_keys[index], params_array[index] if index < params_array.size() else {})
	if index >= chains.size():
		return head
	var chain: Array = chains[index] if chains[index] is Array else []
	if chain.is_empty():
		return head
	var sep: String = _loc(locale, "READ_DIAG_CHAIN_SEP")
	var parts: PackedStringArray = [head]
	for entry: Variant in chain:
		if not entry is Dictionary:
			continue
		var d: Dictionary = entry as Dictionary
		var entry_text: String = _resolve_text(locale, str(d.get("text_key", "")), d.get("params", {}))
		if not entry_text.is_empty():
			parts.append(entry_text)
	return sep.join(parts)


## Format a why-not line: "하려던 것: X — 막힘: Y(param)".
## `why_not` is the dictionary from get_why_not(entity_bits).
static func why_not_line(why_not: Dictionary, locale: Node) -> String:
	var nodes: PackedStringArray = why_not.get("nodes", PackedStringArray())
	var fact_names: PackedStringArray = why_not.get("fact_names", PackedStringArray())
	var actuals: PackedInt64Array = why_not.get("actuals", PackedInt64Array())
	if nodes.is_empty() or fact_names.is_empty():
		return ""
	var wanted: String = _loc(locale, nodes[0])
	if wanted == nodes[0]:
		wanted = nodes[0]
	var blocked: String = _loc(locale, fact_names[0])
	if blocked == fact_names[0]:
		blocked = fact_names[0]
	if not actuals.is_empty():
		var detail: String = str(actuals[0])
		return _loc(locale, "READ_WHY_NOT_FMT").format({"wanted": wanted, "blocked": blocked, "detail": detail})
	return _loc(locale, "READ_WHY_NOT_SHORT_FMT").format({"wanted": wanted, "blocked": blocked})


## Format a request line: "카야: 먹을 게 없어요 — 할 일: 저장 늘리기".
static func request_line(requests: Dictionary, index: int, locale: Node) -> String:
	var text_keys: PackedStringArray = requests.get("text_keys", PackedStringArray())
	var action_keys: PackedStringArray = requests.get("action_keys", PackedStringArray())
	var voice_names: PackedStringArray = requests.get("voice_names", PackedStringArray())
	var steward: Array = requests.get("steward_active", [])
	if index < 0 or index >= text_keys.size():
		return ""
	var voice: String = Names.unbroken(voice_names[index]) if index < voice_names.size() else ""
	var text: String = _loc(locale, text_keys[index])
	var action: String = _loc(locale, action_keys[index]) if index < action_keys.size() else ""
	var is_steward: bool = bool(steward[index]) if index < steward.size() else false
	var status: String = _loc(locale, "READ_STEWARD_DOING") if is_steward else _loc(locale, "READ_STEWARD_TODO")
	return _loc(locale, "READ_PLAQUE_FMT").format({"voice": voice, "text": text, "status": status, "action": action})


## Resolve a text_key with parameters using locale.trf.
static func _resolve_text(locale: Node, key: String, params: Variant) -> String:
	if key.is_empty():
		return ""
	if locale != null and locale.has_method("trf") and params is Dictionary and not (params as Dictionary).is_empty():
		var result: String = str(locale.call("trf", key, params))
		return KoreanText.words(result) if result != key else _loc(locale, key)
	return _loc(locale, key)


## Extract diagnosis for a subject kind from settlement diagnosis chains.
## `subject_prefix` is e.g. "DIAG_STOCKPILE_", "DIAG_HEARTH_", "DIAG_ROOM_".
## Returns "headline ← next" for the first matching chain entry, or "".
static func subject_diagnosis(settlement_diag: Dictionary, subject_prefix: String, locale: Node) -> String:
	var chains: Array = settlement_diag.get("chains", [])
	var text_keys: PackedStringArray = settlement_diag.get("text_keys", PackedStringArray())
	var params_array: Array = settlement_diag.get("params", [])
	for ci: int in chains.size():
		var chain: Array = chains[ci] if ci < chains.size() and chains[ci] is Array else []
		for ei: int in chain.size():
			var entry: Variant = chain[ei]
			if not entry is Dictionary:
				continue
			var entry_key: String = str((entry as Dictionary).get("text_key", ""))
			if entry_key.begins_with(subject_prefix):
				var head: String = _resolve_text(locale, entry_key, (entry as Dictionary).get("params", {}))
				if ei + 1 < chain.size() and chain[ei + 1] is Dictionary:
					var nxt: Dictionary = chain[ei + 1] as Dictionary
					var nxt_text: String = _resolve_text(locale, str(nxt.get("text_key", "")), nxt.get("params", {}))
					if not nxt_text.is_empty():
						return head + _loc(locale, "READ_DIAG_CHAIN_SEP") + nxt_text
				if ci < text_keys.size():
					var stl: String = _resolve_text(locale, text_keys[ci], params_array[ci] if ci < params_array.size() else {})
					if not stl.is_empty() and stl != head:
						return head + _loc(locale, "READ_DIAG_CHAIN_SEP") + stl
				return head
	return ""


static func _loc(locale: Node, key: String) -> String:
	if locale != null and locale.has_method("ltr"):
		return KoreanText.words(str(locale.call("ltr", key)))
	return key
