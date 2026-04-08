extends CanvasModulate
class_name DayNightCycle

## Day/night color cycle driven by the Rust simulation calendar.
##
## Hour formula: hour_of_day = (sim_engine.current_tick % TICKS_PER_DAY) * TICK_HOURS
##   - TICKS_PER_DAY = 12 (GameConfig.TICKS_PER_DAY)
##   - TICK_HOURS = 2    (GameConfig.TICK_HOURS)
##   - Produces 0, 2, 4, …, 22 (12 distinct hours per day)
##
## The Rust harness tests in sim-test/src/main.rs guard the inputs to this
## formula:
##   harness_calendar_ticks_per_day_matches_gdscript (== 12)
##   harness_calendar_hour_formula_consistency       (sequence + monotone)
##   harness_calendar_tick_monotonic                 (no resets)
##
## A CanvasModulate placed in the default canvas layer (Main's children)
## tints world/entities/buildings, but NOT the HUD CanvasLayer (separate
## layer). The HUD remains at full brightness while the game world tints.

const DAY_COLOR: Color = Color(1.0, 1.0, 1.0)        # 08:00–16:00
const DUSK_COLOR: Color = Color(0.95, 0.80, 0.65)    # 16:00–20:00
const NIGHT_COLOR: Color = Color(0.35, 0.35, 0.55)   # 20:00–04:00
const DAWN_COLOR: Color = Color(0.85, 0.75, 0.70)    # 04:00–08:00

var _sim_engine: RefCounted = null


## Wire the day/night cycle to the Rust simulation engine. Call once after
## the engine is constructed.
func setup(sim_engine: RefCounted) -> void:
	_sim_engine = sim_engine


## Recompute the modulate color from the current tick. Call from main.gd's
## _process(). Cheap (one modulo, one int multiply, one piecewise lerp).
func update_cycle() -> void:
	if _sim_engine == null:
		return
	if not "current_tick" in _sim_engine:
		return
	var current_tick: int = int(_sim_engine.current_tick)
	if current_tick < 0:
		return
	var hour: int = (current_tick % GameConfig.TICKS_PER_DAY) * GameConfig.TICK_HOURS
	color = _hour_to_color(hour)


## Translate a discrete game hour (0, 2, 4, …, 22) into a tint color.
## Linear interpolation between the four anchor colors gives smooth transitions.
func _hour_to_color(hour: int) -> Color:
	var h: float = float(hour)
	if h >= 8.0 and h < 16.0:
		return DAY_COLOR
	if h >= 16.0 and h < 20.0:
		var t: float = (h - 16.0) / 4.0
		return DAY_COLOR.lerp(DUSK_COLOR, t)
	if h >= 20.0:
		var t: float = (h - 20.0) / 4.0
		return DUSK_COLOR.lerp(NIGHT_COLOR, t)
	if h < 2.0:
		var t: float = (h + 4.0) / 4.0
		return DUSK_COLOR.lerp(NIGHT_COLOR, t)
	if h >= 2.0 and h < 6.0:
		var t: float = (h - 2.0) / 4.0
		return NIGHT_COLOR.lerp(DAWN_COLOR, t)
	# 6 <= h < 8
	var t: float = (h - 6.0) / 2.0
	return DAWN_COLOR.lerp(DAY_COLOR, t)
