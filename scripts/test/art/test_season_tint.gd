extends SceneTree
const SEASON = preload("res://scripts/ui/art_season_tint.gd")
func _init() -> void:
	assert(SEASON.resolve({"day_index":5,"time_of_day":3.0}) == Color.WHITE)
	assert(SEASON.resolve({"season":2,"days_per_season":0,"day_of_season":0}) == Color.WHITE)
	assert(SEASON.resolve({"season":2,"days_per_season":30}) == Color.WHITE)
	for season: int in range(4):
		var previous: Color = SEASON.TINTS[posmod(season-1,4)]
		var current: Color = SEASON.TINTS[season]
		assert(SEASON.resolve({"season":season,"days_per_season":30,"day_of_season":1,"time_of_day":0.0}) == previous)
		assert(SEASON.resolve({"season":season,"days_per_season":30,"day_of_season":1,"time_of_day":3.0}).is_equal_approx(previous.lerp(current,0.5)))
		assert(SEASON.resolve({"season":season,"days_per_season":30,"day_of_season":1,"time_of_day":6.0}) == current)
		assert(SEASON.resolve({"season":season,"days_per_season":30,"day_of_season":2,"time_of_day":0.0}) == current)
		assert(SEASON.resolve({"season":season,"days_per_season":30,"day_of_season":30,"time_of_day":0.0}) == current)
		for invalid_day: int in [-1, 0, 31]:
			assert(SEASON.resolve({"season":season,"days_per_season":30,"day_of_season":invalid_day,"time_of_day":3.0}) == Color.WHITE)
	assert(SEASON.resolve({"season":0,"days_per_season":7,"day_of_season":2,"time_of_day":0.0}) == SEASON.TINTS[0])
	print("test_season_tint: PASS")
	quit()
