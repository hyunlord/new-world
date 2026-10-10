extends SceneTree
const Topbar = preload("res://scripts/ui/panels/hud_topbar.gd")

func _initialize() -> void:
	call_deferred("_run")

func _run() -> void:
	var topbar: Control = Topbar.new()
	root.add_child(topbar)
	topbar.set_process(false)
	var original_clock: String = topbar._clock_label.text
	assert(topbar._season_label({}) == "")
	assert(topbar._season_label({"season": 0}) == "")
	assert(topbar._season_label({"season": 0, "day_of_season": 0, "days_per_season": 0}) == "")
	assert(topbar._season_label({"season": 0, "day_of_season": 2, "days_per_season": 7}) == " · " + (topbar._loc("HUD_SEASON_DAY_FMT") % [topbar._loc("HUD_SEASON_SPRING"), 2]))
	assert(topbar._season_label({"season": 0, "day_of_season": 0, "days_per_season": 7}) == "")
	assert(topbar._season_label({"season": 0, "day_of_season": -1, "days_per_season": 7}) == "")
	for season: int in 4:
		var keys: Array[String] = ["HUD_SEASON_SPRING", "HUD_SEASON_SUMMER", "HUD_SEASON_AUTUMN", "HUD_SEASON_WINTER"]
		var text: String = topbar._season_label({"season": season, "day_of_season": 1, "days_per_season": 30})
		assert(text == " · " + (topbar._loc("HUD_SEASON_DAY_FMT") % [topbar._loc(keys[season]), 1]))
	assert(topbar._season_label({"season": 3, "day_of_season": 30, "days_per_season": 30}).contains("30"))
	assert(topbar._season_label({"season": 4, "day_of_season": 1, "days_per_season": 30}) == "")
	assert(topbar._season_label({"season": 0, "day_of_season": 31, "days_per_season": 30}) == "")
	topbar._refresh()
	assert(topbar._clock_label.text == original_clock)
	topbar.free()
	print("HUD SEASON PASS: missing/partial keys preserve clock, four localized seasons, one-based first/last day and bridge day2 fixture, invalid values suppressed")
	quit(0)
