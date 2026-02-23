extends RefCounted

## Gregorian calendar system for simulation time conversion.
## 1 tick = 2 hours, 12 ticks/day, 365 days/year (366 leap).

const TICK_HOURS: int = 2
const TICKS_PER_DAY: int = 12
const DAYS_PER_YEAR: int = 365
const TICKS_PER_YEAR: int = 4380  # 365 × 12
const TICKS_PER_MONTH_AVG: int = 365  # ~30.4 days × 12 ticks

const MONTH_DAYS: Array[int] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]


static func is_leap_year(year: int) -> bool:
	return (year % 4 == 0 and year % 100 != 0) or (year % 400 == 0)


static func days_in_year(year: int) -> int:
	return 366 if is_leap_year(year) else 365


## Convert simulation tick to calendar date
static func tick_to_date(tick: int) -> Dictionary:
	var total_days: int = tick / TICKS_PER_DAY
	var hour: int = (tick % TICKS_PER_DAY) * TICK_HOURS
	var minute: int = 0  # 1 tick = 2 hours, so minute stays 0 unless tick granularity changes.

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
		"minute": minute,
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


## Format date for HUD display
static func format_date(tick: int) -> String:
	var d: Dictionary = tick_to_date(tick)
	var season: String = get_season(d.day_of_year)
	return Locale.trf("DATE_FORMAT", {
		"year": d.year,
		"month": Locale.get_month_name(d.month),
		"day": d.day,
		"hour": "%02d" % d.hour,
		"season": Locale.tr_id("SEASON", season),
	})


## Format short date (month + day only) for action history display
static func format_short_date(tick: int) -> String:
	var d: Dictionary = tick_to_date(tick)
	return Locale.trf("UI_SHORT_DATE", {"month": str(d.month), "day": str(d.day)})


## Format short date with year for cross-year display
static func format_short_date_with_year(tick: int) -> String:
	var d: Dictionary = tick_to_date(tick)
	return Locale.trf("UI_SHORT_DATE_WITH_YEAR", {
		"year": str(d.year), "month": str(d.month), "day": str(d.day)
	})


## Format full date + time for HUD (year/month/day + hour:minute)
static func format_full_datetime(tick: int) -> String:
	var d: Dictionary = tick_to_date(tick)
	return Locale.trf("UI_FULL_DATETIME", {
		"year": str(d.year),
		"month": str(d.month),
		"day": str(d.day),
		"hour": "%02d" % d.hour,
		"minute": "%02d" % d.get("minute", 0),
	})


## Format short date + time without year (month/day + hour:minute)
static func format_short_datetime(tick: int) -> String:
	var d: Dictionary = tick_to_date(tick)
	return Locale.trf("UI_SHORT_DATETIME", {
		"month": str(d.month),
		"day": str(d.day),
		"hour": "%02d" % d.hour,
		"minute": "%02d" % d.get("minute", 0),
	})


## Format short date + time with year (year/month/day + hour:minute)
static func format_short_datetime_with_year(tick: int) -> String:
	var d: Dictionary = tick_to_date(tick)
	return Locale.trf("UI_SHORT_DATETIME_WITH_YEAR", {
		"year": str(d.year),
		"month": str(d.month),
		"day": str(d.day),
		"hour": "%02d" % d.hour,
		"minute": "%02d" % d.get("minute", 0),
	})


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
## For negative birth_ticks (pre-game entities), calculates date before Y1.1.1 (game start).
## Correctly handles sub-year ages by computing remaining months, so birth dates
## are never in the future relative to game start.
static func birth_date_from_tick(birth_tick: int, rng: RandomNumberGenerator = null) -> Dictionary:
	if birth_tick >= 0:
		var d: Dictionary = tick_to_date(birth_tick)
		return {"year": d.year, "month": d.month, "day": d.day}
	# Pre-game entity: go backwards from game start (Y1.1.1)
	var age_ticks: int = -birth_tick
	var age_years: int = age_ticks / TICKS_PER_YEAR
	var remaining_ticks: int = age_ticks % TICKS_PER_YEAR
	var remaining_months: int = remaining_ticks / TICKS_PER_MONTH_AVG
	# Subtract full years and remaining months from game start (Y1, month 1)
	var birth_year: int = 1 - age_years
	var birth_month: int = 1 - remaining_months
	# Normalize month underflow (e.g. month 0 → Dec of previous year)
	while birth_month <= 0:
		birth_year -= 1
		birth_month += 12
	var day: int = 1
	if rng != null:
		day = rng.randi_range(1, 28)
	return {"year": birth_year, "month": birth_month, "day": day}


## Format birth_date for UI display
static func format_birth_date(bd: Dictionary) -> String:
	if bd.is_empty():
		return ""
	var year: int = bd.get("year", 0)
	var month: int = bd.get("month", 1)
	var day: int = bd.get("day", 1)
	return Locale.trf("BIRTH_DATE_FORMAT", {
		"year": year,
		"month": Locale.get_month_name(clampi(month, 1, 12)),
		"day": day,
	})


static func to_julian_day(date: Dictionary) -> int:
	var y: int = date.get("year", 1)
	var m: int = date.get("month", 1)
	var d: int = date.get("day", 1)
	if m <= 2:
		y -= 1
		m += 12
	var A: int = y / 100
	var B: int = 2 - A + A / 4
	return int(365.25 * (y + 4716)) + int(30.6001 * (m + 1)) + d + B - 1524


static func count_days_between(from_date: Dictionary, to_date: Dictionary) -> int:
	return absi(to_julian_day(to_date) - to_julian_day(from_date))


static func days_in_month(month: int, year: int) -> int:
	if month == 2:
		return 29 if is_leap_year(year) else 28
	return MONTH_DAYS[clampi(month - 1, 0, 11)]


static func calculate_detailed_age(birth_date: Dictionary, ref_date: Dictionary = {}) -> Dictionary:
	if birth_date.is_empty():
		return {"years": 0, "months": 0, "days": 0, "total_days": 0}
	if ref_date.is_empty():
		ref_date = {"year": 1, "month": 1, "day": 1}

	var years: int = ref_date.get("year", 1) - birth_date.get("year", 1)
	var months: int = ref_date.get("month", 1) - birth_date.get("month", 1)
	var d: int = ref_date.get("day", 1) - birth_date.get("day", 1)

	if d < 0:
		months -= 1
		var prev_month: int = ref_date.get("month", 1) - 1
		var prev_year: int = ref_date.get("year", 1)
		if prev_month <= 0:
			prev_month = 12
			prev_year -= 1
		d += days_in_month(prev_month, prev_year)
	if d < 0:
		d = 0

	if months < 0:
		years -= 1
		months += 12
	if months < 0:
		months = 0

	years = maxi(years, 0)
	var total_days: int = count_days_between(birth_date, ref_date)

	return {"years": years, "months": months, "days": d, "total_days": total_days}


static func format_age_detailed(birth_date: Dictionary, ref_date: Dictionary = {}) -> String:
	var age: Dictionary = calculate_detailed_age(birth_date, ref_date)
	var parts: Array = []
	if age.years > 0:
		parts.append("%d%s" % [age.years, Locale.ltr("UI_AGE_YEARS")])
	if age.months > 0:
		parts.append("%d%s" % [age.months, Locale.ltr("UI_AGE_MONTHS")])
	parts.append("%d%s" % [age.days, Locale.ltr("UI_AGE_DAYS")])
	var age_str: String = " ".join(parts)
	var total_str: String = format_number(age.total_days)
	return "%s %s" % [age_str, Locale.trf("UI_AGE_TOTAL_DAYS_FMT", {"n": total_str})]


static func format_age_short(birth_date: Dictionary, ref_date: Dictionary = {}) -> String:
	var age: Dictionary = calculate_detailed_age(birth_date, ref_date)
	if age.years > 0:
		return Locale.trf("UI_AGE_SHORT_FORMAT", {"y": str(age.years), "m": str(age.months), "d": str(age.days)})
	elif age.months > 0:
		return "%s%s %s%s" % [str(age.months), Locale.ltr("UI_AGE_MONTHS"), str(age.days), Locale.ltr("UI_AGE_DAYS")]
	else:
		return "%s%s" % [str(age.days), Locale.ltr("UI_AGE_DAYS")]


static func format_number(n: int) -> String:
	var s: String = str(absi(n))
	var result: String = ""
	for i in range(s.length()):
		if i > 0 and (s.length() - i) % 3 == 0:
			result += ","
		result += s[i]
	if n < 0:
		result = "-" + result
	return result
