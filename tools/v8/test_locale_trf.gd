# tools/v8/test_locale_trf.gd — unit test of Locale.trf / Locale.format_text (M1 9.7).
# Purpose: placeholder substitution with {name} params on the real locale tables (ko default, then en),
#   including the M1.5 year key UI_TIME_YEAR_FMT as the top bar formats it (String.format), and the M3 quality
#   words (EconomyText.quality, band edges).
# Bridge methods called: none.
# usage: Godot --headless --path . --script res://tools/v8/test_locale_trf.gd
#   prints TRF_TEST_PASS and exits 0, or prints each failure and exits 1.
extends SceneTree

const LocaleScript = preload("res://scripts/core/locale.gd")
const EconomyTextScript = preload("res://scripts/ui/economy_text.gd")

var _failures: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(label: String, got: String, want: String) -> void:
	if got != want:
		_failures += 1
		print("TRF_TEST_FAIL %s: got '%s' want '%s'" % [label, got, want])


func _run() -> void:
	_check("one", LocaleScript.format_text("{count}개", {"count": 12}), "12개")
	_check("two", LocaleScript.format_text("통나무 {delivered}/{needed}", {"delivered": 3, "needed": 12}), "통나무 3/12")
	_check("repeat", LocaleScript.format_text("{a}-{a}", {"a": "x"}), "x-x")
	_check("missing param stays", LocaleScript.format_text("{a} {b}", {"a": 1}), "1 {b}")
	_check("extra param ignored", LocaleScript.format_text("no slots", {"a": 1}), "no slots")
	_check("braces only on names", LocaleScript.format_text("{count", {"count": 5}), "{count")
	var locale: Node = root.get_node_or_null("Locale")
	if locale == null:
		_failures += 1
		print("TRF_TEST_FAIL no Locale autoload")
	else:
		locale.call("set_language", "ko")
		_check("ko table", str(locale.call("trf", "UI_BLUEPRINT_LOGS", {"delivered": 5, "needed": 12})), "통나무 5/12")
		_check("ko count", str(locale.call("trf", "UI_ORDER_RESULT_COUNT", {"count": 15})), "15개")
		_check("ko year", str(locale.call("ltr", "UI_TIME_YEAR_FMT")).format({"year": 1}), "1년 차")
		# M3 goal 3.4: quality words, band edges.
		_check("quality 62", EconomyTextScript.quality(locale, 62), "훌륭 (62)")
		_check("quality 59", EconomyTextScript.quality(locale, 59), "좋음 (59)")
		_check("quality 19", EconomyTextScript.quality(locale, 19), "조악 (19)")
		_check("quality 20", EconomyTextScript.quality(locale, 20), "보통 (20)")
		_check("quality 79", EconomyTextScript.quality(locale, 79), "훌륭 (79)")
		_check("quality 80", EconomyTextScript.quality(locale, 80), "걸작 (80)")
		_check("quality 100", EconomyTextScript.quality(locale, 100), "걸작 (100)")
		_check("quality 0", EconomyTextScript.quality(locale, 0), "조악 (0)")
		_check("stack fmt", EconomyTextScript.stack(locale, 4, "flint", 1, 45), "부싯돌 날 ×1 · 좋음 (45)")
		locale.call("set_language", "en")
		_check("en table", str(locale.call("trf", "UI_BLUEPRINT_LOGS", {"delivered": 5, "needed": 12})), "5/12 logs")
		_check("en year", str(locale.call("ltr", "UI_TIME_YEAR_FMT")).format({"year": 3}), "Year 3")
		_check("missing key", str(locale.call("trf", "NO_SUCH_KEY_M1", {"a": 1})), "NO_SUCH_KEY_M1")
		locale.call("set_language", "ko")
	if _failures == 0:
		print("TRF_TEST_PASS")
		quit(0)
	else:
		quit(1)
