# T-2012-02: 월간 인구 로그

## Objective
Add a monthly population summary log line to the console whenever the in-game month changes.

## Non-goals
- Do NOT modify any files other than mortality_system.gd
- Do NOT change the existing annual demography log
- Do NOT add new signals or events
- Do NOT change any death/birth logic

## File to Modify

### scripts/systems/mortality_system.gd

**Change A: Add monthly tracking variables** (near the yearly tracking vars, around line 32)
```gdscript
## Monthly demography tracking
var _month_births: int = 0
var _month_deaths_starve: int = 0
var _month_deaths_siler: int = 0
var _last_log_month: int = 0
var _last_log_month_year: int = 0
```

**Change B: Add monthly check in execute_tick** (after `_check_annual_demography_log(tick)`)
```gdscript
func execute_tick(tick: int) -> void:
    _check_birthday_mortality(tick)
    _check_infant_monthly(tick)
    _check_monthly_pop_log(tick)  # ADD THIS
    _check_annual_demography_log(tick)
```

**Change C: Add _check_monthly_pop_log function** (new function)
```gdscript
func _check_monthly_pop_log(tick: int) -> void:
    var date: Dictionary = GameCalendar.tick_to_date(tick)
    var current_month: int = date.month
    var current_year: int = date.year
    if current_year == _last_log_month_year and current_month == _last_log_month:
        return
    # New month crossed — print previous month's summary (skip first tick)
    if _last_log_month > 0:
        _print_monthly_pop_log(tick)
    _last_log_month = current_month
    _last_log_month_year = current_year
    # Reset monthly counters
    _month_births = 0
    _month_deaths_starve = 0
    _month_deaths_siler = 0
```

**Change D: Add _print_monthly_pop_log function** (new function)
```gdscript
func _print_monthly_pop_log(tick: int) -> void:
    var alive: Array = _entity_manager.get_alive_entities()
    var total_pop: int = alive.size()
    var adult_count: int = 0
    var child_count: int = 0
    for i in range(alive.size()):
        var entity: RefCounted = alive[i]
        if entity.age_stage == "adult" or entity.age_stage == "elder":
            adult_count += 1
        else:
            child_count += 1
    var date: Dictionary = GameCalendar.tick_to_date(tick)
    print("[POP] Y%d M%d | Pop:%d (Adult:%d Child:%d) | Births:%d | Deaths(starve:%d siler:%d)" % [
        date.year, date.month, total_pop, adult_count, child_count,
        _month_births, _month_deaths_starve, _month_deaths_siler,
    ])
```

**Change E: Update register_death to increment monthly counters** (in the existing register_death function)
At the end of the `register_death` function, add:
```gdscript
# After the existing yearly tracking code, add:
if cause == "starvation":
    _month_deaths_starve += 1
else:
    _month_deaths_siler += 1
```

**Change F: Update register_birth to increment monthly counter** (in the existing register_birth function)
```gdscript
func register_birth() -> void:
    _year_births += 1
    _month_births += 1  # ADD THIS LINE
```

## Acceptance Criteria
- [ ] Monthly population log prints when month changes
- [ ] Format: `[POP] Y%d M%d | Pop:%d (Adult:%d Child:%d) | Births:%d | Deaths(starve:%d siler:%d)`
- [ ] Counters reset at month start
- [ ] Starvation deaths and Siler deaths tracked separately
- [ ] No changes to any other files
- [ ] Existing annual demography log unchanged
- [ ] Code compiles without errors (GDScript syntax valid)

## Context
- GameCalendar is preloaded: `const GameCalendar = preload("res://scripts/core/game_calendar.gd")`
- GameCalendar.tick_to_date(tick) returns: {"year": int, "month": int, "day": int, "hour": int, "day_of_year": int, "tick": int}
- register_death is called by both NeedsSystem (cause="starvation") and MortalitySystem itself (cause="infant_mortality"|"background"|"old_age")
- register_birth is called by FamilySystem
- _entity_manager.get_alive_entities() returns Array of RefCounted entities
- entity.age_stage is one of: "infant", "toddler", "child", "teen", "adult", "elder"
