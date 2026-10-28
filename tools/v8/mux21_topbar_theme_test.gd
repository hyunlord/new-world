# Actual topbar color updates must not invalidate unchanged Control themes.
# Bridge methods: none. Run headless; this is not performance acceptance.
extends SceneTree

const Topbar = preload("res://scripts/ui/hud/hud_topbar.gd")
var _changes: int = 0
var _checks: int = 0
var _failures: int = 0

func _initialize() -> void:
	call_deferred("_run")

func _changed() -> void:
	_changes += 1

func _check(condition: bool, label: String) -> void:
	_checks += 1
	if not condition:
		_failures += 1
	print("MUX21_TOPBAR_THEME %s %s" % ["PASS" if condition else "FAIL", label])

func _run() -> void:
	var topbar: Topbar = Topbar.new()
	var cold: Label = Label.new()
	var risk: Button = Button.new()
	root.add_child(cold)
	root.add_child(risk)
	topbar._cold_label = cold
	topbar._risk_button = risk
	cold.theme_changed.connect(_changed)
	risk.theme_changed.connect(_changed)
	await process_frame
	topbar._refresh_cold_color(1)
	_check(cold.has_theme_color_override("font_color") and cold.get_theme_color("font_color") == Color("a8c0d0"), "cold state applies immediately")
	_changes = 0
	for index: int in 10:
		topbar._refresh_cold_color(1)
	_check(_changes == 0, "unchanged cold state emits no theme changes")
	topbar._refresh_cold_color(0)
	_check(not cold.has_theme_color_override("font_color"), "warm state restores inherited theme immediately")
	_changes = 0
	topbar._refresh_cold_color(0)
	_check(_changes == 0, "unchanged warm state emits no theme changes")
	for count: int in [0, 2, 0]:
		topbar._refresh_risk_color(count)
		var expected: Color = Topbar.RISK_COLOR if count > 0 else Topbar.QUIET_COLOR
		for state: String in ["font_color", "font_hover_color", "font_pressed_color", "font_focus_color"]:
			_check(risk.has_theme_color_override(state) and risk.get_theme_color(state) == expected, "risk transition %d %s applies immediately" % [count, state])
		_changes = 0
		for index: int in 10:
			topbar._refresh_risk_color(count)
		_check(_changes == 0, "unchanged risk %d emits no theme changes" % count)
	# An external theme override must be corrected even when the count is unchanged.
	risk.add_theme_color_override("font_color", Color.MAGENTA)
	topbar._refresh_risk_color(0)
	_check(risk.get_theme_color("font_color") == Topbar.QUIET_COLOR, "external override corrected without stale state cache")
	topbar.free()
	cold.free()
	risk.free()
	print("MUX21_TOPBAR_THEME_DONE " + JSON.stringify({"checks": _checks, "failures": _failures}))
	quit(1 if _failures else 0)
