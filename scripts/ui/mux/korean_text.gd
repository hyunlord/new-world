# Keep Korean words intact at Unicode line-break opportunities in displayed UI text.
# Bridge methods: none. Locale dictionaries remain raw; the idempotent display transform uses WORD JOINER.
extends RefCounted
const JOINER: String = "\u2060"
static func words(text: String) -> String:
	var source: String = text.replace(JOINER, "")
	var result: String = ""
	var previous: bool = false
	for i: int in source.length():
		var code: int = source.unicode_at(i)
		var hangul: bool = (code >= 0xac00 and code <= 0xd7a3) or (code >= 0x1100 and code <= 0x11ff) or (code >= 0x3130 and code <= 0x318f)
		if previous and hangul:
			result += JOINER
		result += source[i]
		previous = hangul
	return result
static func raw(text: String) -> String:
	return text.replace(JOINER, "")
