extends "res://scripts/core/simulation_system.gd"

## Siler(1979) bathtub-curve mortality model.
## μ(x) = a₁·e^{-b₁·x} + a₂ + a₃·e^{b₃·x}
## Birthday-based distributed checks (not every-tick iteration).
## Infants (0-1yr) checked monthly for higher resolution.

const GameCalendar = preload("res://scripts/core/game_calendar.gd")

var _entity_manager: RefCounted
var _rng: RandomNumberGenerator

## Siler baseline parameters (tech=0, hunter-gatherer)
## Target: q0 ≈ 0.40, e0 ≈ 33 years
const SILER: Dictionary = {
	"a1": 0.60,    # infant hazard scale
	"b1": 1.30,    # infant hazard decay speed
	"a2": 0.010,   # background hazard (annual ~1%)
	"a3": 0.00006, # senescence scale
	"b3": 0.090,   # senescence slope (doubling ~7.7yr)
}

## Tech modifier decay rates: m_i(tech) = exp(-k_i * tech)
const TECH_K1: float = 0.30  # infant: tech=10 → m1≈0.05
const TECH_K2: float = 0.20  # background: tech=10 → m2≈0.14
const TECH_K3: float = 0.05  # senescence: tech=10 → m3≈0.61

## Current tech level (0-10, will be driven by research system later)
var tech_level: float = 0.0

## Demography tracking (per-year)
var _year_births: int = 0
var _year_deaths: int = 0
var _year_infant_deaths: int = 0
var _year_start_pop: int = 0
var _last_log_year: int = 0


func _init() -> void:
	system_name = "mortality"
	priority = 49  # After age(48), before population(50)
	tick_interval = 1  # Check every tick (but only process birthday entities)


func init(entity_manager: RefCounted, rng: RandomNumberGenerator) -> void:
	_entity_manager = entity_manager
	_rng = rng


func execute_tick(tick: int) -> void:
	_check_birthday_mortality(tick)
	_check_infant_monthly(tick)
	_check_annual_demography_log(tick)


## ─── Birthday-based annual mortality check ──────────────
## Only entities whose birth_tick aligns with current tick (mod TICKS_PER_YEAR)

func _check_birthday_mortality(tick: int) -> void:
	var tick_mod: int = tick % GameCalendar.TICKS_PER_YEAR
	var alive: Array = _entity_manager.get_alive_entities()
	for i in range(alive.size()):
		var entity: RefCounted = alive[i]
		# Skip infants (handled by monthly check)
		var age_ticks: int = entity.age
		if age_ticks < GameCalendar.TICKS_PER_YEAR:
			continue
		# Birthday check: birth_tick mod TICKS_PER_YEAR == current tick mod TICKS_PER_YEAR
		if entity.birth_tick % GameCalendar.TICKS_PER_YEAR != tick_mod:
			continue
		_do_mortality_check(entity, tick, false)


## ─── Infant monthly mortality check ──────────────────────
## 0-1 year entities checked every ~30 days for higher resolution

func _check_infant_monthly(tick: int) -> void:
	var alive: Array = _entity_manager.get_alive_entities()
	for i in range(alive.size()):
		var entity: RefCounted = alive[i]
		var age_ticks: int = entity.age
		if age_ticks >= GameCalendar.TICKS_PER_YEAR:
			continue
		# Monthly check: every TICKS_PER_MONTH_AVG ticks after birth
		var age_mod: int = age_ticks % GameCalendar.TICKS_PER_MONTH_AVG
		# Check on the tick when age crosses a month boundary
		if age_mod != 0:
			continue
		# Skip tick 0 (just born)
		if age_ticks == 0:
			continue
		_do_mortality_check(entity, tick, true)


## ─── Core mortality calculation ──────────────────────────

func _do_mortality_check(entity: RefCounted, tick: int, is_monthly: bool) -> void:
	var age_years: float = GameConfig.get_age_years(entity.age)

	# Siler hazard components
	var mu_infant: float = SILER.a1 * exp(-SILER.b1 * age_years)
	var mu_background: float = SILER.a2
	var mu_senescence: float = SILER.a3 * exp(SILER.b3 * age_years)

	# Tech modifiers
	var m1: float = exp(-TECH_K1 * tech_level)
	var m2: float = exp(-TECH_K2 * tech_level)
	var m3: float = exp(-TECH_K3 * tech_level)

	# Nutrition modifier (based on hunger)
	var nutrition: float = clampf(entity.hunger, 0.0, 1.0)
	m1 *= lerpf(2.0, 0.8, nutrition)
	m2 *= lerpf(1.5, 0.9, nutrition)

	# Season modifier
	var date: Dictionary = GameCalendar.tick_to_date(tick)
	var season: String = GameCalendar.get_season(date.day_of_year)
	if season == "winter":
		m1 *= 1.3
		m2 *= 1.2
	elif season == "summer":
		m1 *= 0.9

	# Total hazard rate (annual)
	var h_infant: float = m1 * mu_infant
	var h_background: float = m2 * mu_background
	var h_senescence: float = m3 * mu_senescence
	var mu_total: float = h_infant + h_background + h_senescence

	# Individual frailty
	mu_total *= entity.frailty

	# Annual death probability: q = 1 - exp(-μ)
	var q_annual: float = 1.0 - exp(-mu_total)
	q_annual = clampf(q_annual, 0.0, 0.999)

	# Convert to check period probability
	var q_check: float = q_annual
	if is_monthly:
		# Monthly: q_month = 1 - (1 - q_annual)^(1/12)
		q_check = 1.0 - pow(1.0 - q_annual, 1.0 / 12.0)

	# Roll
	if _rng.randf() < q_check:
		var cause: String = _determine_cause(h_infant, h_background, h_senescence)
		_entity_manager.kill_entity(entity.id, cause, tick)
		_year_deaths += 1
		if age_years < 1.0:
			_year_infant_deaths += 1
		emit_event("entity_died_siler", {
			"entity_id": entity.id,
			"entity_name": entity.entity_name,
			"age_years": age_years,
			"cause": cause,
			"mu_total": mu_total,
			"q_annual": q_annual,
			"tick": tick,
		})


## ─── Cause of death determination ───────────────────────

func _determine_cause(h_infant: float, h_background: float, h_senescence: float) -> String:
	var total: float = h_infant + h_background + h_senescence
	if total <= 0.0:
		return "unknown"
	var roll: float = _rng.randf() * total
	if roll < h_infant:
		return "infant_disease"
	elif roll < h_infant + h_background:
		return "accident_or_infection"
	else:
		return "old_age"


## ─── Annual demography log ──────────────────────────────

func _check_annual_demography_log(tick: int) -> void:
	var date: Dictionary = GameCalendar.tick_to_date(tick)
	var current_year: int = date.year
	if current_year <= _last_log_year:
		return
	# New year crossed
	if _last_log_year > 0:
		_print_demography_log(current_year - 1, tick)
	_last_log_year = current_year
	_year_start_pop = _entity_manager.get_alive_count()
	_year_births = 0
	_year_deaths = 0
	_year_infant_deaths = 0


func _print_demography_log(year: int, tick: int) -> void:
	var pop: int = _entity_manager.get_alive_count()
	var q0: float = 0.0
	if _year_births > 0:
		q0 = float(_year_infant_deaths) / float(_year_births)

	# Theoretical e0 from current Siler parameters
	var e0: float = _calc_theoretical_e0()
	var e15: float = _calc_theoretical_ex(15.0)

	# Average age
	var alive: Array = _entity_manager.get_alive_entities()
	var total_age: float = 0.0
	for i in range(alive.size()):
		total_age += GameConfig.get_age_years(alive[i].age)
	var avg_age: float = total_age / maxf(float(alive.size()), 1.0)

	print("[Demography] Y%d: pop=%d births=%d deaths=%d infant_deaths=%d q0=%.2f e0=%.1f e15=%.1f avg_age=%.1f" % [
		year, pop, _year_births, _year_deaths, _year_infant_deaths, q0, e0, e15, avg_age,
	])


## Register a birth (called externally by FamilySystem)
func register_birth() -> void:
	_year_births += 1


## ─── Theoretical life expectancy (numerical integration) ──

func _calc_theoretical_e0() -> float:
	return _calc_theoretical_ex(0.0)


func _calc_theoretical_ex(start_age: float) -> float:
	# Numerical integration of survival function S(x) from start_age
	# e(start) = integral from start to 120 of S(x)/S(start) dx
	var m1: float = exp(-TECH_K1 * tech_level)
	var m2: float = exp(-TECH_K2 * tech_level)
	var m3: float = exp(-TECH_K3 * tech_level)
	var dx: float = 0.5  # half-year steps
	var cum_hazard_start: float = 0.0
	var cum_hazard: float = 0.0
	var integral: float = 0.0

	# Compute cumulative hazard up to start_age
	var x: float = 0.0
	while x < start_age:
		var mu: float = m1 * SILER.a1 * exp(-SILER.b1 * x) + m2 * SILER.a2 + m3 * SILER.a3 * exp(SILER.b3 * x)
		cum_hazard_start += mu * dx
		x += dx

	# Integrate S(x)/S(start) from start_age to 120
	x = start_age
	cum_hazard = 0.0
	while x < 120.0:
		var mu: float = m1 * SILER.a1 * exp(-SILER.b1 * x) + m2 * SILER.a2 + m3 * SILER.a3 * exp(SILER.b3 * x)
		var s_rel: float = exp(-cum_hazard)  # S(x)/S(start)
		integral += s_rel * dx
		cum_hazard += mu * dx
		x += dx

	return integral
