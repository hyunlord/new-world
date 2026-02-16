extends RefCounted

## Gregorian calendar system for simulation time conversion.
## 1 tick = 2 hours, 12 ticks/day, 365 days/year (366 leap).

const TICK_HOURS: int = 2
const TICKS_PER_DAY: int = 12
const DAYS_PER_YEAR: int = 365
const TICKS_PER_YEAR: int = 4380  # 365 × 12
const TICKS_PER_MONTH_AVG: int = 365  # ~30.4 days × 12 ticks

const MONTH_DAYS: Array[int] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
const MONTH_NAMES: Array[String] = [
	"1월", "2월", "3월", "4월", "5월", "6월",
	"7월", "8월", "9월", "10월", "11월", "12월",
]

const SEASON_NAMES: Dictionary = {
	"winter": "겨울",
	"spring": "봄",
	"summer": "여름",
	"autumn": "가을",
}


static func is_leap_year(year: int) -> bool:
	return (year % 4 == 0 and year % 100 != 0) or (year % 400 == 0)


static func days_in_year(year: int) -> int:
	return 366 if is_leap_year(year) else 365


## Convert simulation tick to calendar date
static func tick_to_date(tick: int) -> Dictionary:
	var total_days: int = tick / TICKS_PER_DAY
	var hour: int = (tick % TICKS_PER_DAY) * TICK_HOURS

	# Year calculation (leap-year aware)
	var year: int = 1
	var remaining_days: int = total_days
	while true:
		var dy: int = days_in_year(year)
		if remaining_days < dy:
			break
		remaining_days -= dy
		year += 1

	var day_of_year: int = remaining_days

	# Month/day calculation
	var month_days: Array[int] = MONTH_DAYS.duplicate()
	if is_leap_year(year):
		month_days[1] = 29

	var month: int = 0
	var day_remaining: int = remaining_days
	while month < 12 and day_remaining >= month_days[month]:
		day_remaining -= month_days[month]
		month += 1

	var day: int = day_remaining + 1  # 1-based
	month += 1  # 1-based

	return {
		"year": year,
		"month": month,
		"day": day,
		"hour": hour,
		"day_of_year": day_of_year,
		"tick": tick,
	}


## Get season from day of year (Northern hemisphere)
static func get_season(day_of_year: int) -> String:
	if day_of_year < 59:
		return "winter"      # Jan-Feb
	elif day_of_year < 151:
		return "spring"      # Mar-May
	elif day_of_year < 243:
		return "summer"      # Jun-Aug
	elif day_of_year < 334:
		return "autumn"      # Sep-Nov
	else:
		return "winter"      # Dec


## Format date for HUD display: "Y3 7월 15일 14:00 (여름)"
static func format_date(tick: int) -> String:
	var d: Dictionary = tick_to_date(tick)
	var season: String = get_season(d.day_of_year)
	var season_kr: String = SEASON_NAMES.get(season, season)
	return "Y%d %s %d일 %02d:00 (%s)" % [
		d.year, MONTH_NAMES[d.month - 1], d.day, d.hour, season_kr,
	]


## Convert age in ticks to years (float)
static func tick_to_age_years(birth_tick: int, current_tick: int) -> float:
	return float(current_tick - birth_tick) / float(TICKS_PER_YEAR)


## Get age stage from age in years (6 stages)
static func get_age_stage_from_years(age_years: float) -> String:
	if age_years < 3.0:
		return "infant"
	elif age_years < 6.0:
		return "toddler"
	elif age_years < 12.0:
		return "child"
	elif age_years < 15.0:
		return "teen"
	elif age_years < 56.0:
		return "adult"
	else:
		return "elder"


## Get age stage from age in ticks (6 stages)
static func get_age_stage(age_ticks: int) -> String:
	var age_years: float = float(age_ticks) / float(TICKS_PER_YEAR)
	return get_age_stage_from_years(age_years)


## Days between tick_a and tick_b
static func ticks_to_days(ticks: int) -> int:
	return ticks / TICKS_PER_DAY


## Convert days to ticks
static func days_to_ticks(days: int) -> int:
	return days * TICKS_PER_DAY


## Generate birth_date Dictionary from a birth_tick value.
## For negative birth_ticks (pre-game entities), calculates date before Y1.
static func birth_date_from_tick(birth_tick: int, rng: RandomNumberGenerator = null) -> Dictionary:
	if birth_tick >= 0:
		var d: Dictionary = tick_to_date(birth_tick)
		return {"year": d.year, "month": d.month, "day": d.day}
	# Pre-game entity: born before Y1
	var age_ticks: int = -birth_tick
	var age_years: int = age_ticks / TICKS_PER_YEAR
	var birth_year: int = 1 - age_years
	var month: int = 1
	var day: int = 1
	if rng != null:
		month = rng.randi_range(1, 12)
		day = rng.randi_range(1, 28)
	return {"year": birth_year, "month": month, "day": day}


## Format birth_date for UI display: "Y-25 7월 15일생"
static func format_birth_date(bd: Dictionary) -> String:
	if bd.is_empty():
		return ""
	var year: int = bd.get("year", 0)
	var month: int = bd.get("month", 1)
	var day: int = bd.get("day", 1)
	return "Y%d %s %d일생" % [year, MONTH_NAMES[clampi(month - 1, 0, 11)], day]
